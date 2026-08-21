import { beforeEach, describe, expect, it, vitest } from 'vitest';
import { HttpResponse } from '@angular/common/http';
import { provideHttpClientTesting } from '@angular/common/http/testing';
import { ComponentFixture, TestBed } from '@angular/core/testing';
import { ActivatedRoute } from '@angular/router';

import { provideTranslateService } from '@ngx-translate/core';
import { Subject, from, of } from 'rxjs';

import { ICandidateProfile } from 'app/entities/candidate-profile/candidate-profile.model';
import { CandidateProfileService } from 'app/entities/candidate-profile/service/candidate-profile.service';
import { IJobOffer } from 'app/entities/job-offer/job-offer.model';
import { JobOfferService } from 'app/entities/job-offer/service/job-offer.service';
import { IJobApplication } from '../job-application.model';
import { JobApplicationService } from '../service/job-application.service';

import { JobApplicationFormService } from './job-application-form.service';
import { JobApplicationUpdate } from './job-application-update';

describe('JobApplication Management Update Component', () => {
  let comp: JobApplicationUpdate;
  let fixture: ComponentFixture<JobApplicationUpdate>;
  let activatedRoute: ActivatedRoute;
  let jobApplicationFormService: JobApplicationFormService;
  let jobApplicationService: JobApplicationService;
  let jobOfferService: JobOfferService;
  let candidateProfileService: CandidateProfileService;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [
        provideTranslateService(),
        provideHttpClientTesting(),
        {
          provide: ActivatedRoute,
          useValue: {
            params: from([{}]),
          },
        },
      ],
    });

    fixture = TestBed.createComponent(JobApplicationUpdate);
    activatedRoute = TestBed.inject(ActivatedRoute);
    jobApplicationFormService = TestBed.inject(JobApplicationFormService);
    jobApplicationService = TestBed.inject(JobApplicationService);
    jobOfferService = TestBed.inject(JobOfferService);
    candidateProfileService = TestBed.inject(CandidateProfileService);

    comp = fixture.componentInstance;
  });

  describe('ngOnInit', () => {
    it('should call JobOffer query and add missing value', () => {
      const jobApplication: IJobApplication = { id: 562 };
      const jobOffer: IJobOffer = { id: 9246 };
      jobApplication.jobOffer = jobOffer;

      const jobOfferCollection: IJobOffer[] = [{ id: 9246 }];
      vitest.spyOn(jobOfferService, 'query').mockReturnValue(of(new HttpResponse({ body: jobOfferCollection })));
      const additionalJobOffers = [jobOffer];
      const expectedCollection: IJobOffer[] = [...additionalJobOffers, ...jobOfferCollection];
      vitest.spyOn(jobOfferService, 'addJobOfferToCollectionIfMissing').mockReturnValue(expectedCollection);

      activatedRoute.data = of({ jobApplication });
      comp.ngOnInit();

      expect(jobOfferService.query).toHaveBeenCalled();
      expect(jobOfferService.addJobOfferToCollectionIfMissing).toHaveBeenCalledWith(
        jobOfferCollection,
        ...additionalJobOffers.map(i => expect.objectContaining(i) as typeof i),
      );
      expect(comp.jobOffersSharedCollection()).toEqual(expectedCollection);
    });

    it('should call CandidateProfile query and add missing value', () => {
      const jobApplication: IJobApplication = { id: 562 };
      const candidateProfile: ICandidateProfile = { id: 25911 };
      jobApplication.candidateProfile = candidateProfile;

      const candidateProfileCollection: ICandidateProfile[] = [{ id: 25911 }];
      vitest.spyOn(candidateProfileService, 'query').mockReturnValue(of(new HttpResponse({ body: candidateProfileCollection })));
      const additionalCandidateProfiles = [candidateProfile];
      const expectedCollection: ICandidateProfile[] = [...additionalCandidateProfiles, ...candidateProfileCollection];
      vitest.spyOn(candidateProfileService, 'addCandidateProfileToCollectionIfMissing').mockReturnValue(expectedCollection);

      activatedRoute.data = of({ jobApplication });
      comp.ngOnInit();

      expect(candidateProfileService.query).toHaveBeenCalled();
      expect(candidateProfileService.addCandidateProfileToCollectionIfMissing).toHaveBeenCalledWith(
        candidateProfileCollection,
        ...additionalCandidateProfiles.map(i => expect.objectContaining(i) as typeof i),
      );
      expect(comp.candidateProfilesSharedCollection()).toEqual(expectedCollection);
    });

    it('should update editForm', () => {
      const jobApplication: IJobApplication = { id: 562 };
      const jobOffer: IJobOffer = { id: 9246 };
      jobApplication.jobOffer = jobOffer;
      const candidateProfile: ICandidateProfile = { id: 25911 };
      jobApplication.candidateProfile = candidateProfile;

      activatedRoute.data = of({ jobApplication });
      comp.ngOnInit();

      expect(comp.jobOffersSharedCollection()).toContainEqual(jobOffer);
      expect(comp.candidateProfilesSharedCollection()).toContainEqual(candidateProfile);
      expect(comp.jobApplication).toEqual(jobApplication);
    });
  });

  describe('save', () => {
    it('should call update service on save for existing entity', () => {
      // GIVEN
      const saveSubject = new Subject<IJobApplication>();
      const jobApplication = { id: 20361 };
      vitest.spyOn(jobApplicationFormService, 'getJobApplication').mockReturnValue(jobApplication);
      vitest.spyOn(jobApplicationService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ jobApplication });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(jobApplication);
      saveSubject.complete();

      // THEN
      expect(jobApplicationFormService.getJobApplication).toHaveBeenCalled();
      expect(comp.previousState).toHaveBeenCalled();
      expect(jobApplicationService.update).toHaveBeenCalledWith(expect.objectContaining(jobApplication));
      expect(comp.isSaving()).toEqual(false);
    });

    it('should call create service on save for new entity', () => {
      // GIVEN
      const saveSubject = new Subject<IJobApplication>();
      const jobApplication = { id: 20361 };
      vitest.spyOn(jobApplicationFormService, 'getJobApplication').mockReturnValue({ id: null });
      vitest.spyOn(jobApplicationService, 'create').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ jobApplication: null });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.next(jobApplication);
      saveSubject.complete();

      // THEN
      expect(jobApplicationFormService.getJobApplication).toHaveBeenCalled();
      expect(jobApplicationService.create).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).toHaveBeenCalled();
    });

    it('should set isSaving to false on error', () => {
      // GIVEN
      const saveSubject = new Subject<IJobApplication>();
      const jobApplication = { id: 20361 };
      vitest.spyOn(jobApplicationService, 'update').mockReturnValue(saveSubject);
      vitest.spyOn(comp, 'previousState');
      activatedRoute.data = of({ jobApplication });
      comp.ngOnInit();

      // WHEN
      comp.save();
      expect(comp.isSaving()).toEqual(true);
      saveSubject.error('This is an error!');

      // THEN
      expect(jobApplicationService.update).toHaveBeenCalled();
      expect(comp.isSaving()).toEqual(false);
      expect(comp.previousState).not.toHaveBeenCalled();
    });
  });

  describe('Compare relationships', () => {
    describe('compareJobOffer', () => {
      it('should forward to jobOfferService', () => {
        const entity = { id: 9246 };
        const entity2 = { id: 5985 };
        vitest.spyOn(jobOfferService, 'compareJobOffer');
        comp.compareJobOffer(entity, entity2);
        expect(jobOfferService.compareJobOffer).toHaveBeenCalledWith(entity, entity2);
      });
    });

    describe('compareCandidateProfile', () => {
      it('should forward to candidateProfileService', () => {
        const entity = { id: 25911 };
        const entity2 = { id: 10019 };
        vitest.spyOn(candidateProfileService, 'compareCandidateProfile');
        comp.compareCandidateProfile(entity, entity2);
        expect(candidateProfileService.compareCandidateProfile).toHaveBeenCalledWith(entity, entity2);
      });
    });
  });
});
