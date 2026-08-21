import { HttpResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize, map } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { ICandidateProfile } from 'app/entities/candidate-profile/candidate-profile.model';
import { CandidateProfileService } from 'app/entities/candidate-profile/service/candidate-profile.service';
import { ApplicationStatus } from 'app/entities/enumerations/application-status.model';
import { IJobOffer } from 'app/entities/job-offer/job-offer.model';
import { JobOfferService } from 'app/entities/job-offer/service/job-offer.service';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { IJobApplication } from '../job-application.model';
import { JobApplicationService } from '../service/job-application.service';

import { JobApplicationFormGroup, JobApplicationFormService } from './job-application-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-job-application-update',
  templateUrl: './job-application-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class JobApplicationUpdate implements OnInit {
  readonly isSaving = signal(false);
  jobApplication: IJobApplication | null = null;
  applicationStatusValues = Object.keys(ApplicationStatus);

  jobOffersSharedCollection = signal<IJobOffer[]>([]);
  candidateProfilesSharedCollection = signal<ICandidateProfile[]>([]);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected jobApplicationService = inject(JobApplicationService);
  protected jobApplicationFormService = inject(JobApplicationFormService);
  protected jobOfferService = inject(JobOfferService);
  protected candidateProfileService = inject(CandidateProfileService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: JobApplicationFormGroup = this.jobApplicationFormService.createJobApplicationFormGroup();

  compareJobOffer = (o1: IJobOffer | null, o2: IJobOffer | null): boolean => this.jobOfferService.compareJobOffer(o1, o2);

  compareCandidateProfile = (o1: ICandidateProfile | null, o2: ICandidateProfile | null): boolean =>
    this.candidateProfileService.compareCandidateProfile(o1, o2);

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ jobApplication }) => {
      this.jobApplication = jobApplication;
      if (jobApplication) {
        this.updateForm(jobApplication);
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
    const jobApplication = this.jobApplicationFormService.getJobApplication(this.editForm);
    if (jobApplication.id === null) {
      this.subscribeToSaveResponse(this.jobApplicationService.create(jobApplication));
    } else {
      this.subscribeToSaveResponse(this.jobApplicationService.update(jobApplication));
    }
  }

  protected subscribeToSaveResponse(result: Observable<IJobApplication | null>): void {
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

  protected updateForm(jobApplication: IJobApplication): void {
    this.jobApplication = jobApplication;
    this.jobApplicationFormService.resetForm(this.editForm, jobApplication);

    this.jobOffersSharedCollection.update(jobOffers =>
      this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, jobApplication.jobOffer),
    );
    this.candidateProfilesSharedCollection.update(candidateProfiles =>
      this.candidateProfileService.addCandidateProfileToCollectionIfMissing<ICandidateProfile>(
        candidateProfiles,
        jobApplication.candidateProfile,
      ),
    );
  }

  protected loadRelationshipsOptions(): void {
    this.jobOfferService
      .query()
      .pipe(map((res: HttpResponse<IJobOffer[]>) => res.body ?? []))
      .pipe(
        map((jobOffers: IJobOffer[]) =>
          this.jobOfferService.addJobOfferToCollectionIfMissing<IJobOffer>(jobOffers, this.jobApplication?.jobOffer),
        ),
      )
      .subscribe((jobOffers: IJobOffer[]) => this.jobOffersSharedCollection.set(jobOffers));

    this.candidateProfileService
      .query()
      .pipe(map((res: HttpResponse<ICandidateProfile[]>) => res.body ?? []))
      .pipe(
        map((candidateProfiles: ICandidateProfile[]) =>
          this.candidateProfileService.addCandidateProfileToCollectionIfMissing<ICandidateProfile>(
            candidateProfiles,
            this.jobApplication?.candidateProfile,
          ),
        ),
      )
      .subscribe((candidateProfiles: ICandidateProfile[]) => this.candidateProfilesSharedCollection.set(candidateProfiles));
  }
}
