import { HttpResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize, map } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { IJobOffer } from 'app/entities/job-offer/job-offer.model';
import { JobOfferService } from 'app/entities/job-offer/service/job-offer.service';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { IOfferTailoredResume } from '../offer-tailored-resume.model';
import { OfferTailoredResumeService } from '../service/offer-tailored-resume.service';

import { OfferTailoredResumeFormGroup, OfferTailoredResumeFormService } from './offer-tailored-resume-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-offer-tailored-resume-update',
  templateUrl: './offer-tailored-resume-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class OfferTailoredResumeUpdate implements OnInit {
  readonly isSaving = signal(false);
  offerTailoredResume: IOfferTailoredResume | null = null;

  jobOffersSharedCollection = signal<IJobOffer[]>([]);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected offerTailoredResumeService = inject(OfferTailoredResumeService);
  protected offerTailoredResumeFormService = inject(OfferTailoredResumeFormService);
  protected jobOfferService = inject(JobOfferService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: OfferTailoredResumeFormGroup = this.offerTailoredResumeFormService.createOfferTailoredResumeFormGroup();

  compareJobOffer = (o1: IJobOffer | null, o2: IJobOffer | null): boolean => this.jobOfferService.compareJobOffer(o1, o2);

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ offerTailoredResume }) => {
      this.offerTailoredResume = offerTailoredResume;
      if (offerTailoredResume) {
        this.updateForm(offerTailoredResume);
      }

      this.loadRelationshipsOptions();
    });
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    this.dataUtils.openFile(base64String, contentType);
  }

  setFileData(event: Event, field: string, isImage: boolean): void {
    this.dataUtils.loadFileToForm(event, this.editForm, field, isImage).subscribe({
      error: (err: FileLoadError) =>
        this.eventManager.broadcast(
          new EventWithContent<AlertErrorModel>('jobSearchRustApp.error', { ...err, key: `error.file.${err.key}` }),
        ),
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const offerTailoredResume = this.offerTailoredResumeFormService.getOfferTailoredResume(this.editForm);
    if (offerTailoredResume.id === null) {
      this.subscribeToSaveResponse(this.offerTailoredResumeService.create(offerTailoredResume));
    } else {
      this.subscribeToSaveResponse(this.offerTailoredResumeService.update(offerTailoredResume));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IOfferTailoredResume | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(offerTailoredResume: IOfferTailoredResume): void {
    this.offerTailoredResume = offerTailoredResume;
    this.offerTailoredResumeFormService.resetForm(this.editForm, offerTailoredResume);

    this.jobOffersSharedCollection.update(jobOffers =>
      this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, offerTailoredResume.jobOffer),
    );
  }

  protected loadRelationshipsOptions(): void {
    this.jobOfferService
      .query()
      .pipe(map((res: HttpResponse<IJobOffer[]>) => res.body ?? []))
      .pipe(
        map((jobOffers: IJobOffer[]) =>
          this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, this.offerTailoredResume?.jobOffer),
        ),
      )
      .subscribe((jobOffers: IJobOffer[]) => this.jobOffersSharedCollection.set(jobOffers));
  }
}
