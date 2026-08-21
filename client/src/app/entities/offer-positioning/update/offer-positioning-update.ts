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
import { IOfferPositioning } from '../offer-positioning.model';
import { OfferPositioningService } from '../service/offer-positioning.service';

import { OfferPositioningFormGroup, OfferPositioningFormService } from './offer-positioning-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-offer-positioning-update',
  templateUrl: './offer-positioning-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class OfferPositioningUpdate implements OnInit {
  readonly isSaving = signal(false);
  offerPositioning: IOfferPositioning | null = null;

  jobOffersSharedCollection = signal<IJobOffer[]>([]);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected offerPositioningService = inject(OfferPositioningService);
  protected offerPositioningFormService = inject(OfferPositioningFormService);
  protected jobOfferService = inject(JobOfferService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: OfferPositioningFormGroup = this.offerPositioningFormService.createOfferPositioningFormGroup();

  compareJobOffer = (o1: IJobOffer | null, o2: IJobOffer | null): boolean => this.jobOfferService.compareJobOffer(o1, o2);

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ offerPositioning }) => {
      this.offerPositioning = offerPositioning;
      if (offerPositioning) {
        this.updateForm(offerPositioning);
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
    const offerPositioning = this.offerPositioningFormService.getOfferPositioning(this.editForm);
    if (offerPositioning.id === null) {
      this.subscribeToSaveResponse(this.offerPositioningService.create(offerPositioning));
    } else {
      this.subscribeToSaveResponse(this.offerPositioningService.update(offerPositioning));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IOfferPositioning | null>): void {
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

  protected updateForm(offerPositioning: IOfferPositioning): void {
    this.offerPositioning = offerPositioning;
    this.offerPositioningFormService.resetForm(this.editForm, offerPositioning);

    this.jobOffersSharedCollection.update(jobOffers =>
      this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, offerPositioning.jobOffer),
    );
  }

  protected loadRelationshipsOptions(): void {
    this.jobOfferService
      .query()
      .pipe(map((res: HttpResponse<IJobOffer[]>) => res.body ?? []))
      .pipe(
        map((jobOffers: IJobOffer[]) =>
          this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, this.offerPositioning?.jobOffer),
        ),
      )
      .subscribe((jobOffers: IJobOffer[]) => this.jobOffersSharedCollection.set(jobOffers));
  }
}
